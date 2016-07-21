#!/usr/bin/env bash

set -e

# environment variables
cp /vagrant/vagrant/vagrant_env.sh /etc/profile.d/
/etc/profile.d/vagrant_env.sh

# build folder
# cargo shouldn't share items b/t VM and host (using editor build-on-save, for example)
mkdir -p /rust-dashboard/target
chown -R vagrant:vagrant /rust-dashboard/

# dependencies
update-locale LANGE=en_US.UTF-8
locale-gen en_US.UTF-8
apt-get update
apt-get install -y postgresql libpq-dev npm nodejs curl
ln -s /usr/bin/nodejs /usr/bin/node

# setup postgres
sudo -u postgres pg_dropcluster --stop $PGVERSION main
sudo -u postgres pg_createcluster --locale en_US.UTF-8 -e UTF8 --start $PGVERSION main
sudo -u postgres createuser --createdb vagrant
sudo -u vagrant createdb -E UTF8 -l en_US.UTF8 -T template0 dashboard
sudo -u postgres psql -d dashboard -f /vagrant/bootstrap.sql
cp /vagrant/vagrant/pg_hba.conf /etc/postgresql/9.5/main/
echo "listen_addresses = '*'" >> /etc/postgresql/$PGVERSION/main/postgresql.conf
systemctl restart postgresql

# install rust
#curl https://sh.rustup.rs -sSf | sudo -u vagrant sh -s -- -y
#sudo -u vagrant source $HOME/.cargo/env
#sudo -u vagrant echo "PATH=$PATH:$HOME/.cargo/bin" >> $HOME/.bashrc
#sudo -u vagrant rustup default $RUST_NIGHTLY_VERSION
#sudo -u vagrant rustup update

# for DB migrations
sudo -u vagrant cargo install diesel_cli --no-default-features --features "postgres"

# frontend deps
npm install -g ember-cli
npm install -g bower
