(ns potboiler.tree
  (:require
   [pandect.algo.sha256 :refer :all]
   [clj-leveldb :as level]
   [puget.printer :as puget]
  [clojure.edn :as edn])
)

(defmacro def- [id value]
  `(def ^{:public false} ~id ~value)
)

(def- master "MASTER")

(def- start "START")

(def- id "ID")

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

(defn parentkey [db key]
  (-> (getobject db key) :parent)
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

(defn dbid [db]
  (level/get db id)
)

(defn uuid [] (str (java.util.UUID/randomUUID)))

(def hashfn sha256)

(defn initdb [db]
  (let [identifier (uuid)
        startval (uuid)
        startkey (hashfn startval)]
    (level/put db
               id identifier
               start startkey
               startkey startval
               master startkey)
  )
)

(defn additem [db value]
  (let [canonvalue (tostr value)
        valuehash (hashfn canonvalue)
        oldmaster (masterkey db)
        storage {:hash valuehash :value canonvalue :parent oldmaster}
        masterhash (sha256 (str valuehash oldmaster))
        ]
    (level/put db
               masterhash (tostr storage)
               master masterhash)
  )
)
