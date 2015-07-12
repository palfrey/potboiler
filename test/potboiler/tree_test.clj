(ns potboiler.tree-test
  (:use midje.sweet)
  (:require
   [potboiler.tree :refer :all]
   [clj-leveldb :as level]
   [me.raynes.fs :as fs]
   [clojure.tools.logging :as log])
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

(defmacro do-with-db-pair [bodyfn]
  `(let [startval# (uuid)]
    (do-with-dbs 2 [startval# startval#] ~bodyfn)
  )
)

(do-with-db-pair
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

(do-with-db-pair
  (fn [db1 db2]
    (fact "two dbs have same startkey" (startkey db1) => (startkey db2))
    (let [nodes (atom (apply merge (map #(hash-map (dbid %) %) [db1 db2])))
          mk-sendfn (fn [src] (fn [dest msg]
                                (do
                                  (log/debugf "Sending message %s from %s -> %s" msg src dest)
                                  (->> dest (get @nodes) :recvfn (#(% src msg)))
                                )))
          mk-recvfn (fn [dest] (fn [src msg]
                                 (do
                                   (log/debugf "Receiving message %s from %s to %s" msg src dest)
                                   (->> dest (get @nodes) (#(add-from-other % msg)))
                                  )
                                )
                      )
          n1 (-> db1
                 (#(set-sender % (mk-sendfn (dbid %))))
                 (#(set-receiver % (mk-recvfn (dbid %))))
                 (#(add-neighbour % (dbid db2))))
          n2 (-> db2
                 (#(set-sender % (mk-sendfn (dbid %))))
                 (#(set-receiver % (mk-recvfn (dbid %))))
                 (#(add-neighbour % (dbid db1))))
          ]
      (compare-and-set! nodes @nodes (apply merge (map #(hash-map (dbid %) %) [n1 n2])))
      (additem n1 "blah")
      (fact "additem gets sent to other" (masterkey n1) => (masterkey n2))

      (additem db1 "foo") ; adds item to non-sending db to simulate disconnected
      (additem n1 "foo2") ; and then add one to the sending db
      (fact "nodes can be updated if they get out of date" (masterkey n1) => (masterkey n2))
    )
  )
)
