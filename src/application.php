<?php

use \Symfony\Component\HttpFoundation\Request;
use \Symfony\Component\HttpFoundation\Response;
use \Symfony\Component\HttpFoundation\BinaryFileResponse;
use \Symfony\Component\HttpKernel\Exception\NotFoundHttpException;

$app = require __DIR__ . '/bootstrap.php';

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
        $parts[$i] = array($url, $parts[$i]);
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

function generateIndex($root, $path, $level = 0)
{
    $summary = '';

    if (empty($path)) {
        return $summary;
    }

    $indent = str_pad(' ', $level * 4);
    foreach (new \Sanpi\SortableDirectoryIterator($path) as $fileInfo) {
        $filename = $fileInfo->getFilename();
        $title = str_replace('.md', '', $filename);

        if ($filename{0} === '.' || $filename === 'index.md') {
            continue;
        }

        if ($fileInfo->isDir() || isMarkdownFile($fileInfo->getPathname())) {
            $url = "/$root/" . urlencode($filename);
            $summary .= "$indent* [$title]($url)\n";
        }
        if ($fileInfo->isDir()) {
            $summary .= generateIndex("$root/$filename", $fileInfo->getPathname(), $level + 1);
        }
    }
    return $summary;
}

function isMarkdownFile($filename)
{
    return (is_file($filename) && preg_match('/\.md$/', $filename) === 1);
}

$app->get('{slug}', function($slug, Request $request) use($app) {
    $response = null;
    $root = $app['config']['root'];
    $page = urldecode("$root/$slug");

    if (is_file($page) && !isMarkdownFile($page)) {
        $response = new BinaryFileResponse($page);
    }
    else {
        if (is_dir($page)) {
            $index = "$page/index.md";
            if (is_file($index)) {
                $contents .= "> " . file_get_contents($index) . "\n";
            }

            $contents .= generateIndex($slug, $page);
        }
        elseif (is_file($page)) {
            if (isMarkdownFile($page)) {
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
            $response = $app['twig']->render('index.html.twig', array(
                'nav' => generateBreadcrumb($slug),
                'title' => generateTitle($app['config']['title'], $slug),
                'contents' => $contents,
            ));
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
