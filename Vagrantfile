# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-16.04"

  config.vm.provision("apt",
                      type: "shell",
                      path: "vagrant/native_deps.sh",
                      keep_color: true)

  config.vm.provision("postgres",
                      type: "shell",
                      path: "vagrant/postgres.sh",
                      keep_color: true,
                      privileged: false,
                      env: { 'PGVERSION' => '9.5', })

  config.vm.provision("rust",
                      type: "shell",
                      path: "vagrant/rust.sh",
                      keep_color: true,
                      privileged: false,
                      env: { 'RUST_NIGHTLY_VERSION' => 'nightly-2016-06-15', },)

  config.vm.network :forwarded_port, guest: 4200, host: 4040
  config.vm.network :forwarded_port, guest: 5432, host: 4050
end
