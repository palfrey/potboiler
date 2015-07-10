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

(defmacro do-with-dbs
  ([many body] `(do-with-dbs ~many (map (fn [x#] (uuid)) (range ~many)) ~body))
  ([many uuids body]
    `(let [db-dirs# (map (fn [x#] (fs/temp-dir "level-db-test")) (range ~many))
        dbs# (map #(loaddb %1 %2) db-dirs# ~uuids)]
      (apply ~body dbs#)
      (non-lazy-for [db# dbs#] (closedb db#))
      (non-lazy-for [dir# db-dirs#] (level/destroy-db dir#))
    )
  )
)

(defmacro do-with-db [body]
  `(do-with-dbs 1 ~body)
)

(do-with-db
  (fn [db]
    (fact "init sets start" (startkey db) =not=> nil)
    (fact "init sets master" (masterkey db) =not=> nil)
    (fact "init sets id" (dbid db) =not=>  nil)
    (fact "initial db has master == start" (startkey db) => (masterkey db))
  )
)

(do-with-db
  (fn [db]
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

(let [startval (uuid)]
  (do-with-dbs 2 [startval startval]
    (fn [db1 db2]
      (fact "two dbs have same startkey" (startkey db1) => (startkey db2))

      (additem db1 "bar")
      (additem db2 "bar")

      (fact "two dbs with same added items have same master" (masterkey db1) => (masterkey db2))

      (let [msgstore (atom [])
            senddb (set-sender db1 (fn [dest msg] (compare-and-set! msgstore @msgstore (conj @msgstore {:dest dest :msg msg}))))
            neighbourdb (add-neighbour senddb (dbid db2))]

        (fact "additem sends messages to one node" (additem neighbourdb "foo") => 1)
        (fact "message get sent" (count @msgstore) => 1)
        (fact "message gets sent to the right node" (-> @msgstore first :dest) => (dbid db2))
        (fact "message has right parent" (-> @msgstore first :msg :parent) => (masterkey db2))
      )
    )
  )
)
