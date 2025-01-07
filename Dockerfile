# Étape 1 : Image de compilation
FROM clux/muslrust:stable as builder

# Copie des sources
WORKDIR /home/rust/src
COPY . .

# Compilation du projet pour la cible x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

# Étape 2 : Image finale minimaliste
FROM alpine:latest

# Copier le binaire depuis l'étape de compilation
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/monitor_server /usr/local/bin/monitor_server

# Définir le point d'entrée
ENTRYPOINT ["monitor_server"]