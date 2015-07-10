(ns potboiler.tree-test
  (:use midje.sweet)
  (:require
   [potboiler.tree :refer :all]
   [clj-leveldb :as level]
   [me.raynes.fs :as fs])
)

(defmacro do-with-db [body]
  `(let [db-dir# (fs/temp-dir "level-db-test")
      db# (makedb db-dir#)]
    (~body db#)
    (closedb db#)
    (level/destroy-db db-dir#)
  )
)

(do-with-db
 (fn [db]
   (fact "empty db has no master" (masterkey db) => nil)
   (fact "empty db has no start key" (startkey db) => nil)
   )
)

(do-with-db
  (fn [db]
    (initdb db)

    (fact "init sets start" (startkey db) =not=> nil)
    (fact "init sets master" (masterkey db) =not=> nil)
    (fact "initial db has master == start" (startkey db) => (masterkey db))
   )
)

(do-with-db
 (fn [db]
   (initdb db)
   (additem db "foo")

   (fact "one key db has master != start" (startkey db) =not=> (masterkey db))
   (fact "items are retrievable" (masteritem db) => "foo")
   (fact "parents are discoverable" (parentkey db (masterkey db)) => (startkey db))

   (let [simplekey (masterkey db)]
     (additem db {:a "bar"})

     (fact "complex items are retrievable" (masteritem db) => {:a "bar"})
     (fact "historical items are retrievable" (getitem db simplekey) => "foo")
   )
 )
)
