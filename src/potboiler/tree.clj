(ns potboiler.tree
  (:require
   [pandect.algo.sha256 :refer :all]
   [clj-leveldb :as level]
   [puget.printer :as puget]
  [clojure.edn :as edn])
)

(def master "MASTER")

(def start "START")

(defn makedb [dir]
  (level/create-db dir {
                 :key-decoder byte-streams/to-string
                 :val-decoder byte-streams/to-string
                 })
)

(defn closedb [db]
  (.close db)
)

(defn tostr [value]
  (puget/pprint-str value)
)

(defn fromstr [value]
  (edn/read-string value)
)

(defn- getobject [db key]
  (-> (level/get db key) fromstr)
)

(defn getitem [db key]
  (-> (getobject db key) :value fromstr)
)

(defn parentkeys [db key]
  (-> (getobject db key) :parents)
)

(defn masterkey [db]
  (level/get db master)
)

(defn masteritem [db]
  (getitem db (masterkey db))
)

(defn startkey [db]
  (level/get db start)
)

(defn- uuid [] (str (java.util.UUID/randomUUID)))

(def hashfn sha256)

(defn initdb [db]
  (let [startval (uuid)
        startkey (hashfn startval)]
    (level/put db
               start startkey
               startkey startval
               master startkey)
  )
)

(defn additem [db value]
  (let [canonvalue (tostr value)
        valuehash (hashfn canonvalue)
        oldmaster (masterkey db)
        storage {:hash valuehash :value canonvalue :parents [oldmaster]}
        masterhash (sha256 (str valuehash oldmaster))
        ]
    (level/put db
               masterhash (tostr storage)
               master masterhash)
  )
)
