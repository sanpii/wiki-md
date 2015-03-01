<?php

namespace Sanpi;

class File
{
    private $info;

    public function __construct($filename)
    {
        $this->info = new \SplFileInfo($filename);
    }

    public function isMarkdown()
    {
        return $this->hasExtension(['md']);
    }

    public function isImage()
    {
        return $this->hasExtension(['jpg', 'jpeg', 'png', 'gif']);
    }

    public function isVideo()
    {
        return $this->hasExtension(['mpeg', 'ogv', 'mp4']);
    }

    public function isSound()
    {
        return $this->hasExtension(['ogg', 'mp3']);
    }

    public function hasExtension(array $ext)
    {
        return (
            $this->isFile()
            && preg_match('/\.(' . implode('|', $ext) . ')$/i', $this->getFilename()) === 1
        );
    }

    public function __call($name, $arguments)
    {
        return call_user_func_array([$this->info, $name], $arguments);
    }
}