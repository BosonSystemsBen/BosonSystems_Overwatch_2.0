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
- `crates/api` — serveur HTTP (Axum) exposant les modules
- `migration` — migrations de schéma (SeaORM)

D'autres crates (`pennylane`, `sendcloud`, `shipping`, `worker`) seront ajoutés
module par module.

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
