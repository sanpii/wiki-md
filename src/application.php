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

function generateIndex($root, $path, $media)
{
    $summary = '';

    if (empty($path)) {
        return $summary;
    }

    if ($media === true) {
        $summary .= '<ul class="media">';
    }
    else {
        $summary .= '<ul>';
    }

    foreach (new \Sanpi\SortableDirectoryIterator($path) as $fileInfo) {
        $path = $fileInfo->getPathname();
        $filename = $fileInfo->getFilename();
        $url = "/$root/" . urlencode($filename);
        $title = ucfirst(str_replace('.md', '', $filename));

        if ($filename{0} === '.' || $filename === 'index.md') {
            continue;
        }

        if ($media === true) {
            if ($fileInfo->isDir()) {
                $summary .= "<li class=\"folder\" data-content=\"$title\"><a href=\"$url\"><img src=\"/thumbnail$url\" /></a></li>";
            }
            elseif (isImage($path)) {
                $summary .= "<li><a href=\"$url\"><img src=\"/thumbnail$url\" /></a></li>";
            }
            elseif (isSound($path)) {
                $summary .= "<li data-content=\"$title\"><a href=\"$url\"><i class=\"glyphicon glyphicon-music\"></i></a></li>";
            }
            elseif (isVideo($path)) {
                $summary .= "<li data-content=\"$title\"><a href=\"$url\"><i class=\"glyphicon glyphicon-film\"></i></a></li>";
            }
        }
        elseif ($fileInfo->isDir() || isMarkdownFile($path)) {
            $summary .= "<li><a href=\"$url\">$title</a>";
        }

        if ($media === false && $fileInfo->isDir()) {
            $summary .= generateIndex("$root/$filename", $path, $media);
        }

        if ($fileInfo->isDir() || isMarkdownFile($path) || isMedia($path)) {
            $summary .= "</li>";
        }
    }
    $summary .= '</ul>';
    return $summary;
}

function isMarkdownFile($filename)
{
    return (is_file($filename) && preg_match('/\.md$/', $filename) === 1);
}

function isImage($filename)
{
    return (is_file($filename) && preg_match('/\.(jpg|jpeg|png|gif)/i', $filename) === 1);
}

function isVideo($filename)
{
    return (is_file($filename) && preg_match('/\.(mpeg|ogv|mp4)/i', $filename) === 1);
}

function isSound($filename)
{
    return (is_file($filename) && preg_match('/\.(ogg|mp3)/i', $filename) === 1);
}

function isMedia($filename)
{
    return (isImage($filename) || isVideo($filename) || isSound($filename));
}

$app->get('/thumbnail/{slug}', function ($slug, Request $request) use($app) {
    $root = getRootDirectory($app['config'], $request);
    $page = urldecode("$root/$slug");

    if (is_dir($page)) {
        foreach (new \Sanpi\SortableDirectoryIterator($page) as $fileInfo) {
            if (isImage($fileInfo->getPathname())) {
                $page .= "/{$fileInfo->getFilename()}";
                break;
            }
        }
    }

    if (!isImage($page)) {
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

    if (is_file($page) && !isMarkdownFile($page)) {
        $response = new BinaryFileResponse($page);
    }
    else {
        $contents = '';
        $isIndex = false;

        if (is_dir($page)) {
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
            $contents .= generateIndex($slug, $page, $media);
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
