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

## A venir

- un vrai path pour config
- l'emplacement status.html à renseigner dans la configuration

### synchronisation avec api OVH

test

## Dernières modifications

### Authentification par htpasswd

- Mise en place d'un middleware pour sécuriser les routes avec une authentification basique.
- Utilisation de la fonction load_htpasswd pour charger les utilisateurs et mots de passe depuis un fichier htpasswd.
- Comparaison des identifiants fournis avec les mots de passe hachés stockés dans htpasswd.
- Intégration de logs pour suivre les étapes d'authentification.
- fonctionne avec l'outil htaccess d'apache pour générer le fichier ( dans /etc/monitor_server/htpasswd )

### API et fichier de confguration

- Ajout de la route dynamique /status/{service} pour vérifier l'état des services via l'API.
- Gestion des statuts HTTP :
  - 200 OK pour les services actifs.
  - 500 Internal Server Error pour les services inactifs.
- Amélioration des logs pour suivre les appels API.
  Le ficher de configuration est à mettre à coté du binaire, mais prochainement patché

### Scinde le main.rs

### Prise en charge des proxys (Nginx)

- Ajout d'un middleware pour normaliser les chemins d'URL et gérer les en-têtes transmis par les proxys (`X-Forwarded-For`, `X-Forwarded-Proto`).
- Modification de la fonction `get_status` pour utiliser les en-têtes HTTP afin de récupérer et afficher l'adresse IP réelle du client (provenant de `X-Forwarded-For`).

### Journalisation des requêtes

- Intégration d'un middleware `Logger` pour journaliser les requêtes HTTP entrantes, facilitant le débogage et le suivi des accès.

### Informations sur le noyau :

- Ajout de la fonction `get_kernel_version` pour récupérer la version du noyau à l'aide de la commande `uname -r`.
- Affichage des informations du noyau dans l'interface utilisateur sous la section "Kernel".

### Durée d'activité (Uptime) :

- Ajout de la fonction `get_uptime` pour calculer et afficher la durée depuis le dernier démarrage en lisant `/proc/uptime`.
- Les données sont formatées en jours, heures, minutes et secondes pour une meilleure lisibilité.

### Adresses IP :

- Ajout de la fonction `get_ip_addresses` pour récupérer l'adresse IP privée et publique de la machine.
- Ces informations sont affichées dans l'interface utilisateur sous la section "Network".

### Amélioration de l'interface HTML :

- Ajout d'une section dédiée pour afficher l'uptime.
- Mise à jour des styles CSS pour maintenir une présentation claire.

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

## API de vérification des services

**Route : /status/{service}**

Cette route permet de vérifier l'état d'un service spécifique en renvoyant un code HTTP correspondant à son statut.

- Code 200 : Le service est actif.
- Code 500 : Le service est inactif.

## Exemples d'utilisation

- **curl pour statut SSH (actif) et Nginx (inactif)** :

  Service actif :

  ```bash
  curl -i http://127.0.0.1:8080/status/ssh
  Réponse :
  HTTP/1.1 200 OK
  Service 'ssh' is active
  ```

  Service inactif :

  ```bash
  curl -i http://127.0.0.1:8080/status/nginx
  Réponse :
  HTTP/1.1 500 Internal Server Error
  Service 'nginx' is not active
  ```

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
