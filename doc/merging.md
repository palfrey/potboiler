Merging of a new commit
-----------------------
1. Node A sends new commit (struct which includes hash, parent, content) to node B
2. If node B's current master is the parent of the new commit, commit it to B, and finish
   * If it's the same master, just finish
3. Otherwise, B sends back it's current master commit to A
4. If A has that commit in it's history, then A catches B up to it's current master via a series of step 1 and 2's, and then finish
5. Otherwise, A doesn't have that commit, so we're into divergent territory.
6. A sends to B the parent of the current location (start from master on the first run around)
7. If B has that commit, we've found the common ancestor, otherwise goto step 6, but swapping A and B this time.
   * If we reach the start of a node's tree, something has gone wrong. These two nodes are unmergable because they've got different starts!
8. Common ancestor has now been found. "Winning" node is now determined by alphabetical ordering of master hashes. Lowest "wins".
9. Winning node now applies all the extra commits that have been provided in the order given by the losing node.
10. Losing node reverts all it's commits back to the common ancestor, applies all the commits of the winning node up to the point, then applies it's own.
11. (Optional): Node A sends master hash to B to make sure they now agree.

Wash, rinse, repeat for all other nodes. A single node should only be doing a merge past step 5 with one other node at a time, but once this is done, the two merged nodes can now merge with two different nodes, and so on, allowing for speedy merging.