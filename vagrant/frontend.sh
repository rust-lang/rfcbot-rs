#!/usr/bin/env bash

set -e

# global deps
npm install -g ember-cli

# keep node modules in VM file system to speedup npm install and fix it on Windows/Linux machines
mkdir -p /home/vagrant/local_front/node_modules
rm -f /vagrant/front/node_modules || true
ln -s /home/vagrant/local_front/node_modules /vagrant/front/
chown -R vagrant:vagrant /home/vagrant/local_front
chown -R vagrant:vagrant /vagrant/front/node_modules

# install local deps
su vagrant <<'EOF'
cd /vagrant/front
yarn install
EOF
