# Wiki.md

Enhanced directory listing:

* Render markdown file;
* Directory with a ``.media`` file is displayed as thumbnails.

## Installation

    $ git clone https://github.com/sanpii/wiki-md.git
    $ curl http://getcomposer.org/installer | php

## Configuration

    $ cp src/config/{development,current}.php

## Run

### Development

    $ php -S localhost:8080 -t web/ web/index.php

### Production

    $ cat /etc/nginx/sites-enable/wiki
    server {
        listen 80;
        listen 443;
        server_name wiki.homecomputing.fr;
        root /home/git/public_html/wiki-md/web;
        index index.php;

        location / {
            try_files $uri $uri/ @rewrite;
        }

        location @rewrite {
            rewrite ^/(.*)$ /index.php/$1;
        }

        location ~ \.php(/|$) {
            include /etc/nginx/fastcgi_params;
            fastcgi_pass unix:/var/run/php5-fpm/git;
        }
    }
