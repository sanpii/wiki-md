<?php

namespace Sanpi;

class SortableDirectoryIterator implements \IteratorAggregate
{
    private $storage;

    public function __construct($path)
    {
        $this->storage = new \ArrayObject();
        $files = new \DirectoryIterator($path);

        foreach ($files as $file) {
            $this->storage->offsetSet($file->getFilename(), $file->getFileInfo());
        }

        $this->storage->uksort(function ($a, $b) {
            return strcasecmp($a, $b);
        });
    }

    public function getIterator()
    {
        return $this->storage->getIterator();
    }
}
