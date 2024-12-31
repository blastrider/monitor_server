# System Status Dashboard

## Description

Ce programme est une application web écrite en Rust, utilisant le framework `Actix-web`. Il génère une interface web qui affiche des informations sur l'état du système local, notamment la mémoire, le disque, le réseau, les températures, les conteneurs Docker, et le statut SSH.

L'interface est construite en HTML avec des styles sobres pour une lisibilité optimale et une distinction visuelle entre les états importants, comme le statut SSH.

## Fonctionnalités

- **Statut système** :
  - Version de l'OS récupérée via `/etc/os-release`.
  - Nom de l'hôte affiché dynamiquement.
- **Mémoire** :
  - Quantité totale et utilisée en unités lisibles (GB, MB, KB).
- **Disque** :
  - Espace total et disponible sur la partition racine (`/`).
- **Réseau** :
  - Trafic entrant et sortant depuis `/proc/net/dev`.
- **Températures** :
  - Moyenne des températures des capteurs disponibles dans `/sys/class/thermal/`.
- **SSH** :
  - Vérifie si le service `ssh` ou `sshd` est actif.
  - Affiche un statut visuel coloré (vert pour actif, rouge pour inactif).
- **Docker** :
  - Liste des conteneurs Docker avec leur image et état.
- **Interface web** :
  - Interface propre et responsive avec une mise en page basée sur `flexbox`.

## Dépendances

- **Rust et Cargo** : Assurez-vous d'avoir Rust installé sur votre système. Pour cela, visitez [https://www.rust-lang.org/](https://www.rust-lang.org/).
- **Actix-web** : Framework pour construire des API et applications web.
- **Askama** : Utilisé pour générer les templates HTML.
- **Bollard** : Client Docker en Rust.
- **Fern** : Pour la gestion des logs.
- **Hostname** : Pour récupérer le nom d'hôte du système.

## Installation

1. Clonez le dépôt :

   ```bash
   git clone <URL_DU_DEPOT>
   cd <NOM_DU_DOSSIER>
   ```

2. Assurez-vous que les dépendances Docker sont bien configurées pour que `Bollard` puisse se connecter au démon Docker.

3. Ajoutez les dépendances au fichier `Cargo.toml` :

   ```toml
    [dependencies]
    actix-service = "2.0.2"
    actix-web = "4.9.0"
    askama = "0.12.1"
    bollard = "0.18.1"
    chrono = "0.4.39"
    env_logger = "0.11.6"
    fern = "0.7.1"
    hostname = "0.4.0"
    libc = "0.2.169"
    log = "0.4.22"
    serde = { version = "1.0.217", features = ["derive"] }
    tokio = { version = "1.42.0", features = ["full"] }

   ```

4. Compilez et lancez le programme :

   ```bash
   cargo run
   ```

5. Accédez à l'interface via [http://127.0.0.1:8080/status](http://127.0.0.1:8080/status).

## Structure des fichiers

- **`main.rs`** : Contient la logique principale pour récupérer les informations système et les rendre accessibles via l'interface web.
- **`templates/status.html`** : Template HTML pour afficher les informations. Inclut des styles CSS pour une meilleure présentation.

## Fonctionnement

- **Récupération des données système** :
  - Les données sont collectées depuis des fichiers système Linux comme `/proc/meminfo`, `/proc/net/dev`, et `/sys/class/thermal/`.
  - La commande `systemctl` est utilisée pour vérifier l'état des services SSH.
- **Logs** :
  - Les erreurs et informations sont loguées dans un fichier `server.log` et affichées dans la console.
- **Interface utilisateur** :
  - Les données collectées sont rendues dynamiquement dans une page HTML grâce à `Askama`.

## Exemples d'utilisation

- **Vérification rapide du statut SSH** :
  - La section SSH montre si le service est actif avec une indication visuelle (texte vert ou rouge).
- **Suivi des ressources système** :
  - Mémoire, espace disque, et trafic réseau affichés avec des unités lisibles.
- **Surveillance Docker** :
  - Liste les conteneurs Docker en cours d'exécution, leur image et leur état.

## Limitations

- Le programme est conçu pour les systèmes Linux et dépend de fichiers comme `/proc/meminfo` et `/etc/os-release`.
- Certaines fonctionnalités (comme les températures) peuvent ne pas fonctionner sur des systèmes sans capteurs compatibles.

## Contribution

Les contributions sont les bienvenues ! Pour signaler un problème ou proposer une amélioration, ouvrez une issue ou un pull request sur le dépôt GitHub.
