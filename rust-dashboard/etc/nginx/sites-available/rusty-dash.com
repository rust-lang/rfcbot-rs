server {
	listen 80;
	listen [::]:80;

	server_name rusty-dash.com www.rusty-dash.com;

	root /usr/share/rust-dashboard/www;
	index index.html;

	location / {
		try_files $uri $uri/ =404;
	}

    location /api/ {
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_pass http://localhost:8080/;
    }
}
