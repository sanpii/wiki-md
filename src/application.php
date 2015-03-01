<?php

use \Symfony\Component\HttpFoundation\Request;
use \Symfony\Component\HttpFoundation\Response;
use \Symfony\Component\HttpFoundation\BinaryFileResponse;
use \Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

$app = require __DIR__ . '/bootstrap.php';

function getRootDirectory($config, Request $request)
{
    $root = $config['root'];

    if (is_array($root)) {
        $site = $request->server->get('HTTP_HOST');

        if (isset($root[$site])) {
            $root = $root[$site];
        }
        else {
            $root = reset($root);
        }
    }
    return $root;
}

function generateTitle($appTitle, $path)
{
    $parts = explode('/', $path);
    $parts = array_reverse($parts);
    $parts[] = $appTitle;
    return implode(' | ', $parts);
}

function generateBreadcrumb($path)
{
    $parts = explode('/', $path);
    array_unshift($parts, '~');

    $breadcrumb = '';

    for ($i = 0; $i < count($parts); $i++) {
        if ($i === 0) {
            $url = '';
        }
        else {
            $url = $parts[$i - 1][0] . '/' . $parts[$i];
        }
        $parts[$i] = [$url, $parts[$i]];
    }

    foreach ($parts as $part) {
        $url = $part[0];
        $title = $part[1];

        if ($url === '') {
            $url = '/';
        }
        $breadcrumb .= "/[$title]($url)";
    }
    return ltrim($breadcrumb, '/');
}

function generateMedia($app, $root, $path)
{
    $summary = '';

    if (empty($path)) {
        return $summary;
    }

    $files = [];

    foreach (new \Sanpi\SortableDirectoryIterator($path) as $id => $file) {
        if ($id{0} === '.' || $id === 'index.md') {
            continue;
        }

        $url = "/$root/" . urlencode($id);
        $path = $file->getPathName();

        $files[] = [
            'id' => $id,
            'url' => $url,
            'info' => $file,
            'title' => ucfirst(str_replace('.md', '', $id)),
            'thumbnail' => "/thumbnail$url",
        ];
    }

    return $app['twig']->render('panel.html.twig', compact('files'));
}

function generateIndex($root, $path)
{
    $summary = '';

    if (empty($path)) {
        return $summary;
    }

    $files = [];

    $summary .= "<ul>";
    foreach (new \Sanpi\SortableDirectoryIterator($path) as $id => $file) {
        if ($id{0} === '.' || $id === 'index.md') {
            continue;
        }

        $url = "/$root/" . urlencode($id);
        $title = ucfirst(str_replace('.md', '', $id));

        if ($file->isDir() || $file->isMarkdown()) {
            $summary .= "<li><a href=\"$url\">$title</a>";
        }

        if ($file->isDir()) {
            $filename = $file->getFilename();
            $summary .= generateIndex("$root/$filename", $file->getPathName());
        }

        if ($file->isDir() || $file->isMarkdown()) {
            $summary .= "</li>";
        }
    }
    $summary .= "</ul>";

    return $summary;
}

$app->get('/thumbnail/{slug}', function ($slug, Request $request) use($app) {
    $root = getRootDirectory($app['config'], $request);
    $page = urldecode("$root/$slug");
    $file = new \Sanpi\File($page);

    if ($file->isDir()) {
        foreach (new \Sanpi\SortableDirectoryIterator($page) as $file) {
            if ($file->isImage()) {
                $page .= "/{$file->getFilename()}";
                break;
            }
        }
    }

    if (!$file->isImage()) {
        $page = __DIR__ . '/../web/img/missing.png';
    }

    $image = $app['imagine']->open($page)
        ->thumbnail(
            new \Imagine\Image\Box(200, 200),
            \Imagine\Image\ImageInterface::THUMBNAIL_OUTBOUND
        )
        ->show('png');

    return new Response($image, 200, ['Content-Type' => 'image/png']);
})
->value('slug', '.')
->assert('slug', '^[^_].+');

$app->get('{slug}', function($slug, Request $request) use($app) {
    $response = null;
    $root = getRootDirectory($app['config'], $request);
    $page = urldecode("$root/$slug");
    $file = new \Sanpi\File($page);

    if ($file->isFile() && !$file->isMarkdown()) {
        $response = new BinaryFileResponse($page);
    }
    else {
        $media = false;
        $contents = '';
        $isIndex = false;

        if ($file->isDir()) {
            $isIndex = true;

            $index = "$page/index.md";
            if (is_file($index)) {
                $summary = file_get_contents($index);

                foreach (explode("\n", $summary) as $line) {
                    $contents .= "> $line\n";
                }
                $contents .= "\n";
            }

            $media = is_file("$page/.media");
            if ($media === true) {
                $contents .= generateMedia($app, $slug, $page);
            }
            else {
                $contents .= generateIndex($slug, $page);
            }
        }
        elseif ($file->isFile()) {
            if ($file->isMarkdown()) {
                $contents .= file_get_contents($page);
            }
            else {
                $response = new BinaryFileResponse($page);
            }
        }
        else {
            throw new NotFoundHttpException("/$slug not found");
        }

        $accept = explode(',', $request->server->get('HTTP_ACCEPT'));
        if (in_array('text/html', $accept)) {
            $response = $app['twig']->render('index.html.twig', [
                'is_index' => (!$media && $isIndex),
                'nav' => generateBreadcrumb($slug),
                'title' => generateTitle($app['config']['title'], $slug),
                'contents' => $contents,
            ]);
        }
        else {
            $response = new Response($contents, 200, ['Content-Type' => 'text/plain']);
        }

    }
    return $response;
})
->value('slug', '.')
->assert('slug', '^[^_].+');

return $app;
