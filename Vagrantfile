# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-16.04"
  config.vm.provision(:shell,
                      path: "vagrant/bootstrap.sh",
                      env: {
                          'RUST_NIGHTLY_VERSION' => 'nightly-2016-06-15',
                          'PGVERSION' => '9.5',
                      },
                      keep_color: true,)
  config.vm.network :forwarded_port, guest: 4200, host: 4040
end
