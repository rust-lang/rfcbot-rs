#!/usr/bin/env bash

set -e

# environment variables
cp /vagrant/vagrant/vagrant_env.sh /etc/profile.d/

# build folder
# cargo shouldn't share items b/t VM and host (using editor build-on-save, for example)
mkdir -p /rust-dashboard/target
chown -R vagrant:vagrant /rust-dashboard/

# dependencies
update-locale LANGE=en_US.UTF-8
locale-gen en_US.UTF-8
apt-get update
apt-get install -y postgresql libpq-dev npm nodejs curl git pkg-config
ln -s /usr/bin/nodejs /usr/bin/node
