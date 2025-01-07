#!/bin/bash

# Nom du binaire et version
BINARY_NAME="monitor_server"  # Remplacez par le nom réel de votre binaire
BINARY_NAME_package="monitor-server"
VERSION="${1:-1.0.0}"      # Version de l'application, par défaut "1.0.0"
IMAGE_NAME="monitor_server:$VERSION"

# Répertoire temporaire pour le paquet .deb
DEB_DIR="./deb_package"
CONTROL_FILE="$DEB_DIR/DEBIAN/control"

# Étape 1 : Construction de l'image Docker
echo "🚀 Construction de l'image Docker version $VERSION..."
docker build -t "$IMAGE_NAME" .

# Étape 2 : Création du conteneur temporaire
echo "📦 Création d'un conteneur temporaire..."
CONTAINER_ID=$(docker create "$IMAGE_NAME")

# Étape 3 : Extraction du binaire sur la machine locale
echo "📂 Extraction du binaire $BINARY_NAME..."
docker cp "$CONTAINER_ID:/usr/local/bin/$BINARY_NAME" "./$BINARY_NAME"

# Suppression du conteneur temporaire
echo "🧹 Suppression du conteneur temporaire..."
docker rm "$CONTAINER_ID"

# Vérification de l'extraction du binaire
if [[ ! -f "./$BINARY_NAME" ]]; then
    echo "❌ Échec de l'extraction du binaire."
    exit 1
fi

# Donner les permissions d'exécution au binaire
chmod +x "./$BINARY_NAME"

# Étape 4 : Préparation du paquet .deb
echo "📦 Création de l'arborescence pour le paquet .deb..."
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/local/bin"

# Copie du binaire dans l'arborescence
cp "./$BINARY_NAME/$BINARY_NAME" "$DEB_DIR/usr/local/bin/$BINARY_NAME"

# Création du fichier de contrôle
cat > "$CONTROL_FILE" <<EOL
Package: $BINARY_NAME_package
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Maxime Guillemin <guimaxali@gmail.com>
Description: Application monitor server en .deb

EOL

# Construction du paquet .deb
echo "📦 Construction du paquet .deb..."
dpkg-deb --build "$DEB_DIR"

# Renommage du paquet .deb
mv "$DEB_DIR.deb" "${BINARY_NAME}_${VERSION}_amd64.deb"

echo "✅ Paquet .deb créé avec succès : ${BINARY_NAME}_${VERSION}_amd64.deb"
