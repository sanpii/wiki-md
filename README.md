# Spore

[![Build Status](https://travis-ci.org/sanpii/spore.png?branch=master)](https://travis-ci.org/sanpii/spore)
[![Dependencies Status](https://www.wakuwakuw.com/d/7519698)](http://depending.in/sanpii/spore)

Check other branches for create a specific application:

* rest: create a REST API;
* security: create a web application with user management.

## Installation

    $ curl http://getcomposer.org/installer | php
    $ php composer.phar -sdev create-project sanpi/spore

## Configuration

    $ cd src/config
    $ ln -s development.php current.php

## Run

    $ php -S localhost:8080 -t web/ web/index.php

## Test

### Unitary

    $ ./bin/atoum

### Functionally

    $ cp behat.yml{-dist,}
    $ ./bin/behat

## Components

### Silex

> A PHP micro-framework standing on the shoulder of giants

http://silex.sensiolabs.org/

### Pomm

> Pomm is a lightweight, fast, efficient and powerful PHP Object Model Manager
> for the Postgresql relational database.

http://pomm.coolkeums.org/

### Bootstrap

> Sleek, intuitive, and powerful front-end framework for faster and easier web
> development.

http://twitter.github.com/bootstrap/

### Atoum

> The simple, modern and intuitive unit testing framework for PHP 5.3+

http://atoum.org/

### Behat

> A php framework for testing your business expectations.

http://behat.org/
