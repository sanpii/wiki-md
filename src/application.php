<?php

use \Symfony\Component\HttpFoundation\Request;
use \Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

$app = require __DIR__ . '/bootstrap.php';

function generateSummary($root, $path, $level = 0)
{
    $summary = '';

    if (empty($path)) {
        return $summary;
    }

    $indent = str_pad(' ', $level * 4);
    foreach (new DirectoryIterator($path) as $fileInfo) {
        $filename = $fileInfo->getFilename();
        $title = str_replace('.md', '', $filename);

        if ($filename{0} === '.') {
            continue;
        }

        $summary .= "$indent* [$title](/$root/$filename)\n";
        if ($fileInfo->isDir()) {
            $summary .= generateSummary("$root/$filename", $fileInfo->getPathname(), $level + 1);
        }
    }
    return $summary;
}

$app->get('{slug}', function($slug) use($app) {
    $root = $app['config']['root'];
    $page = "$root/$slug";

    $contents = "# /$slug\n";
    if (is_dir($page)) {
        $contents .= generateSummary($slug, $page);
    }
    elseif (is_file($page)) {
        $contents .= file_get_contents($page);
    }
    else {
        throw new NotFoundHttpException("/$slug not found");
    }
    return $app['twig']->render('index.html.twig', array(
        'title' => $app['config']['title'],
        'contents' => $app['parser']->transformMarkdown($contents)
    ));
})
->value('slug', '.')
->assert('slug', '.+');

return $app;
