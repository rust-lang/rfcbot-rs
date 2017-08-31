server {
        listen 80;
        listen [::]:80;

        server_name rusty-dash.com www.rusty-dash.com;
        return 301 https://$server_name$request_uri;
}

server {
        listen 443 ssl http2 default_server;
        listen [::]:443 ssl http2 default_server;

        location / {
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_pass http://localhost:8000/;
        }

        ssl_certificate /etc/letsencrypt/live/rusty-dash.com/fullchain.pem;
        ssl_certificate_key /etc/letsencrypt/live/rusty-dash.com/privkey.pem;

        ssl_protocols TLSv1 TLSv1.1 TLSv1.2;
        ssl_prefer_server_ciphers on;
        ssl_ciphers "EECDH+AESGCM:EDH+AESGCM:AES256+EECDH:AES256+EDH";
        ssl_ecdh_curve secp384r1;
        ssl_session_cache shared:SSL:10m;
        ssl_session_tickets off;
        ssl_stapling on;
        ssl_stapling_verify on;
        resolver 8.8.8.8 8.8.4.4 valid=300s;
        resolver_timeout 5s;
        # Disable preloading HSTS for now.  You can use the commented out header line that includes
        # the "preload" directive if you understand the implications.
        #add_header Strict-Transport-Security "max-age=63072000; includeSubdomains; preload";
        add_header Strict-Transport-Security "max-age=63072000; includeSubdomains";
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;

        ssl_dhparam /etc/ssl/certs/dhparam.pem;
}
