# Wiki.md

## Installation

    $ curl http://getcomposer.org/installer | php
    $ php composer.phar -sdev create-project sanpi/spore

## Configuration

    $ cd src/config
    $ ln -s development.php current.php

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
