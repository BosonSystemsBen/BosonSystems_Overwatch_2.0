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
- `crates/sendcloud` — client HTTP vers l'API v3 Sendcloud (création et
  annonce d'un colis, récupération de l'étiquette)
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
  `labeled`, `shipped`) — toujours modifiable manuellement
- `POST /shipment-lines/:id/link-serial-number` — associer un S/N existant à
  une ligne de préparation (renseigne automatiquement `assigned_to` avec le
  nom du client de la commande)

Le mapping produit Pennylane ↔ nomenclature se fait via `pennylane_product_id`
sur `products` ; une ligne de facture sans produit reconnu, ou sans produit du
tout (remise, texte libre), est ignorée lors de l'import.

## Module Sendcloud (étiquetage)

Nécessite `SENDCLOUD_PUBLIC_KEY`, `SENDCLOUD_PRIVATE_KEY`,
`SENDCLOUD_SENDER_ADDRESS_ID` (adresse d'expédition pré-configurée dans le
panel Sendcloud) et `SENDCLOUD_SHIPPING_OPTION_CODE` (voir `.env.example`).
Sans ces 4 valeurs, l'endpoint répond une erreur claire plutôt que de planter.

- `POST /shipments/:id/create-label` — `{"weight_kg": 1.2}` annonce le colis
  auprès du transporteur, enregistre le tracking et le lien de l'étiquette,
  et passe le statut de la commande à `labeled`. Idempotent : rejouer l'appel
  sur une commande déjà étiquetée renvoie l'étiquette existante sans repayer
  le transporteur (`external_reference_id` = l'id de la commande côté
  Sendcloud, qui dédoublonne).
- Le PDF de l'étiquette (base64) est renvoyé une seule fois, dans la réponse
  de création — récupère-le ou le lien `label_link` à ce moment-là si tu veux
  l'archiver ; les appels suivants ne renvoient plus l'étiquette déjà créée.

⚠️ Chaque appel réussi crée un colis facturé chez le transporteur (annulable
dans le délai indiqué par Sendcloud). Pour tester sans frais, utilise le code
`sendcloud:letter` comme `SENDCLOUD_SHIPPING_OPTION_CODE`.
