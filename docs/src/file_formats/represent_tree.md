# Represent Tree

There is many way to represent a tree on disk, A tree can be use for indexing database records (such as BPlusTree).

## Using Pointer

This is the best way to represent a tree on disk. It use pointer of next block on disk.

| Index | Element | Left | Right |
| :---: | :-----: | :--: | :---: |
|   0   |   --    |  --  |   5   |
|   1   |    A    |  0   |   0   |
|   2   |    B    |  1   |   3   |
|   3   |    C    |  0   |   0   |
|   4   |    D    |  2   |   0   |
|   5   |    E    |  4   |   6   |
|   6   |    F    |  0   |   0   |

To represents this tree:
```
             E
            / \
           /   \
          /     \
         D       F
        /
       /
       B
      / \
     /   \
    /     \
   A       C
```

- Advantages:
  - Easy structure in which to search
  - Easy to insert Easy to delete
  - Easy to read tree back in from disk after writing out (no recreation of
    links required)
  - The programmer can link unused table entries into a "free list." and can
    write functions to allocate and deallocate entries in the table for use as
    tree nodes.

- Disadvantages:
  - Memory allocation is not truly dynamic and it can be difficult to match the
    array size with the size range of the tree.


