(ns potboiler.tree-test
  (:use midje.sweet)
  (:require
   [potboiler.tree :refer :all]
   [clj-leveldb :as level]
   [me.raynes.fs :as fs])
)

(defmacro non-lazy-for [& body]
  `(doall (for ~@body))
)

(defmacro do-with-dbs [many body]
  `(let [db-dirs# (map (fn [x#] (fs/temp-dir "level-db-test")) (range ~many))
      dbs# (map #(makedb %) db-dirs#)]
    (apply ~body dbs#)
    (non-lazy-for [db# dbs#] (closedb db#))
    (non-lazy-for [dir# db-dirs#] (level/destroy-db dir#))
  )
)

(defmacro do-with-db [body]
  `(do-with-dbs 1 ~body)
)

(do-with-db
 (fn [db]
   (fact "empty db has no master" (masterkey db) => nil)
   (fact "empty db has no start key" (startkey db) => nil)
   (fact "empty db has no identity" (dbid db) => nil)
   )
)

(do-with-db
  (fn [db]
    (initdb db)

    (fact "init sets start" (startkey db) =not=> nil)
    (fact "init sets master" (masterkey db) =not=> nil)
    (fact "init sets id" (dbid db) =not=>  nil)
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

(do-with-dbs 2
 (fn [db1 db2]
   (let [startval (uuid)]
     (initdb db1 startval)
     (initdb db2 startval)

     (fact "two dbs have same startkey" (startkey db1) => (startkey db2))

     (additem db1 "bar")
     (additem db2 "bar")

     (fact "two dbs with same added items have same master" (masterkey db1) => (masterkey db2))
    )
  )
)
