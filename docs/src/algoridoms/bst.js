class BST {
    static insert(node, id) {
        if (node.id > id) BST.insert(node.left ??= { id }, id);
        if (node.id < id) BST.insert(node.right ??= { id }, id);
    }
    insert(id) { BST.insert(this, id) }
}
let bst = new BST;
bst.id = 4;
bst.insert(2)
bst.insert(6)
bst.insert(1)
bst.insert(3)
bst.insert(5)
bst.insert(7)
console.log(JSON.stringify(bst, null, 4))

//   Balanced: (4, 2, 6, 1, 3, 5, 7),
//   SemiBalanced: (4, 3, 5, 2, 6, 1, 7),
//   Worst_min: (1, 2, 3, 4, 5, 6, 7),
//   Worst_max: (7, 6, 5, 4, 3, 2, 1),