<?php

use \Ciconia\Ciconia;
use \Ciconia\Extension\Gfm;
use \Silex\Provider\TwigServiceProvider;
use \Silex\Provider\WebProfilerServiceProvider;
use \Silex\Provider\UrlGeneratorServiceProvider;
use \Silex\Provider\ServiceControllerServiceProvider;

require_once __DIR__ . '/../vendor/autoload.php';

if (!is_file(__DIR__ . '/config/current.php')) {
    throw new \RunTimeException('No current configuration file found in config.');
}

$app = new Silex\Application();

$app['config'] = require __DIR__ . '/config/current.php';

$app['debug'] = $app['config']['debug'];

$app['parser'] = function () {
    $parser = new Ciconia();
    $parser->addExtension(new Gfm\FencedCodeBlockExtension());
    $parser->addExtension(new Gfm\TaskListExtension());
    $parser->addExtension(new Gfm\InlineStyleExtension());
    $parser->addExtension(new Gfm\WhiteSpaceExtension());
    $parser->addExtension(new Gfm\TableExtension());
    $parser->addExtension(new Gfm\UrlAutoLinkExtension());
    return $parser;
};

$app['imagine'] = function () {
    return new \Imagine\Imagick\Imagine();
};

$app->register(new TwigServiceProvider(), [
    'twig.path' => __DIR__ . '/views',
]);

$app['twig'] = $app->share($app->extend('twig', function($twig, $app) {
    $transform = function($string) use($app) {
        return $app['parser']->render($string);
    };

    $twig->addFilter(
        'markdown',
        new \Twig_Filter_Function($transform, ['is_safe' => ['html']])
    );

    return $twig;
}));

if (class_exists('\Silex\Provider\WebProfilerServiceProvider')) {
    $app->register(new UrlGeneratorServiceProvider());
    $app->register(new ServiceControllerServiceProvider());

    $profiler = new WebProfilerServiceProvider();
    $app->register($profiler, [
        'profiler.cache_dir' => __DIR__ . '/../cache/profiler',
    ]);
    $app->mount('/_profiler', $profiler);
}

return $app;
