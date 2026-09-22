# BosonSystems Overwatch 2.0

Dashboard modulaire interne pour la gestion de la boîte : extraction des factures
Pennylane, préparation d'expédition, gestion des numéros de série, étiquetage
Sendcloud. Pennylane reste la source de vérité pour les clients/factures ;
Overwatch ajoute ce qui manque pour l'expédition.

## Architecture

Workspace Cargo multi-crates :

- `crates/common` — types partagés, config, erreurs
- `crates/nomenclature` — produits (nomenclature) et numéros de série, avec
  détection automatique du produit à partir d'un S/N scanné
- `crates/pennylane` — client HTTP vers l'API Entreprise Pennylane (factures,
  lignes de facture, clients)
- `crates/shipping` — commandes à préparer (shipments) importées depuis
  Pennylane, liées à la nomenclature et aux S/N
- `crates/sendcloud` — client HTTP vers l'API v3 Sendcloud (Orders API,
  pour validation humaine avant étiquetage)
- `crates/api` — serveur HTTP (Axum) exposant les modules
- `migration` — migrations de schéma (SeaORM)

D'autres crates (`worker`) seront ajoutés module par module.

## Démarrer en local

```bash
cp .env.example .env
docker compose up -d postgres
cargo run --bin migrate -- up
cargo run --bin api
```

L'API écoute sur `http://localhost:8080`.

## Authentification

Toute l'API/UI est protégée par Basic Auth (`APP_USERNAME`/`APP_PASSWORD`,
requis — l'app refuse de démarrer sans), sauf `/health` (pour les contrôles
de santé Fly). Le navigateur affiche une invite native au premier accès.

## Module Nomenclature & Numéros de série

- `GET/POST /products` — lister/créer un produit (SKU, nom, `requires_serial`,
  `detection_pattern` : regex optionnelle pour la détection automatique)
- `GET/PUT/DELETE /products/:id` — consulter/éditer/supprimer un produit
  (la suppression échoue si des S/N y sont encore rattachés)
- `GET /serial-numbers?product_id=&status=` — lister les S/N (filtres optionnels)
- `POST /serial-numbers` — enregistrer un S/N pour un produit
- `POST /serial-numbers/:id/assign` — associer un S/N à un client avant packing
- `DELETE /serial-numbers/:id` — supprimer un S/N
- `POST /serial-numbers/detect` — `{"value": "..."}` renvoie le ou les produits
  dont le `detection_pattern` matche la valeur scannée

Tout (nomenclature, règles de détection, associations) est éditable à tout
moment via ces endpoints.

## Module Pennylane & Préparation d'expédition

Nécessite `PENNYLANE_API_TOKEN` (voir `.env.example`). Un produit doit avoir
son `pennylane_product_id` renseigné (`PUT /products/:id`) pour que ses
lignes de facture soient automatiquement rattachées à la nomenclature.

- `POST /pennylane/sync` — importe les factures clients finalisées (non
  brouillon) depuis Pennylane comme `shipments` à préparer, avec leurs lignes
  d'articles et l'adresse de livraison du client
- `GET /shipments` — lister les commandes à préparer
- `GET /shipments/:id` — détail d'une commande avec ses lignes
- `PUT /shipments/:id/status` — changer le statut (`pending`, `ready`,
  `sent_to_sendcloud`, `shipped`) — toujours modifiable manuellement
- `POST /shipment-lines/:id/link-serial-number` — associer un S/N existant à
  une ligne de préparation (renseigne automatiquement `assigned_to` avec le
  nom du client de la commande)

Le mapping produit Pennylane ↔ nomenclature se fait via `pennylane_product_id`
sur `products` ; une ligne de facture sans produit reconnu, ou sans produit du
tout (remise, texte libre), est ignorée lors de l'import.

## Module Sendcloud (validation humaine avant expédition)

Nécessite `SENDCLOUD_PUBLIC_KEY`, `SENDCLOUD_PRIVATE_KEY` et
`SENDCLOUD_INTEGRATION_ID` (voir `.env.example`). Ce dernier vient d'une
"Integration" créée dans le panel Sendcloud (Settings > Integrations),
requise par l'API Orders. Sans ces 3 valeurs, l'endpoint répond une erreur
claire plutôt que de planter.

Le système **ne crée jamais d'étiquette ni ne déclenche de facturation
transporteur automatiquement** — il pousse juste une "order" chez Sendcloud
pour qu'un humain la complète (infos douane si besoin) et crée l'étiquette
manuellement depuis le panel Sendcloud. C'est volontaire : une déclaration
douane automatique mal remplie peut bloquer un colis, donc la décision finale
reste toujours humaine.

- `POST /shipments/:id/send-to-sendcloud` — `{"weight_kg": 1.2}` pousse la
  commande (adresse, lignes, montants) comme order Sendcloud et passe le
  statut à `sent_to_sendcloud`. Idempotent : rejouer l'appel sur une commande
  déjà envoyée renvoie l'order existant plutôt que d'en créer un doublon
  (upsert Sendcloud sur `order_id` = l'id de la commande côté nous).
- Les montants par ligne viennent du champ `amount` de la facture Pennylane
  (`amount_eur` sur `shipment_lines`), pour que l'humain ait de quoi remplir
  la douane sans retourner sur Pennylane.
