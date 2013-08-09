<?php

use \Symfony\Component\HttpFoundation\Request;

$app = require __DIR__ . '/bootstrap.php';

$app->get('/', function(Request $request) use($app) {
    return $app['twig']->render('index.html.twig');
});

return $app;
