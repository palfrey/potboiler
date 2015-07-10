(ns potboiler.tree
  (:require
   [pandect.algo.sha256 :refer :all]
   [clj-leveldb :as level]
   [puget.printer :as puget]
   [clojure.edn :as edn]
   [clojure.tools.logging :as log]
  )
)

(defmacro def- [id value]
  `(def ^{:public false} ~id ~value)
)

(def- master "MASTER")

(def- start "START")

(def- id "ID")

(defn set-sender [db newfn]
  (assoc db :sendfn newfn)
)

(defn add-neighbour [db neighbour-id]
  (assoc db :nodes (conj (:nodes db) neighbour-id))
)

(defn- getdb [db]
  (:db db)
)

(defn closedb [db]
  (-> db getdb .close)
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
  (-> (getobject (getdb db) key) :value fromstr)
)

(defn parentkey [db key]
  (-> (getobject (getdb db) key) :parent)
)

(defn masterkey [db]
  (level/get (getdb db) master)
)

(defn masteritem [db]
  (getitem db (masterkey db))
)

(defn startkey [db]
  (level/get (getdb db) start)
)

(defn dbid [db]
  (level/get (getdb db) id)
)

(defn uuid [] (str (java.util.UUID/randomUUID)))

(def hashfn sha256)

(defn- initdb
  ([db] (initdb db (uuid)))
  ([db startval]
    (let [identifier (uuid)
          startkey (hashfn startval)]
      (level/put db
                 id identifier
                 start startkey
                 startkey startval
                 master startkey)
    )
  )
)

(defn loaddb
  ([dir] (loaddb dir (uuid)))
  ([dir startval]
    (let [db (level/create-db dir {
                   :key-decoder byte-streams/to-string
                   :val-decoder byte-streams/to-string
                   })]
      (if (nil? (dbid {:db db}))
        (initdb db startval)
      )
      {:db db :nodes []
        :sendfn (fn [dest msg]
          (log/infof "Sending message %s to %s" msg dest)
          true
        )
      }
    )
  )
)

(defn additem [db value]
  (let [canonvalue (tostr value)
        valuehash (hashfn canonvalue)
        oldmaster (masterkey db)
        storage {:hash valuehash :value canonvalue :parent oldmaster}
        masterhash (sha256 (str valuehash oldmaster))
        ]
    (level/put (getdb db)
               masterhash (tostr storage)
               master masterhash)
    (reduce + 0 (map #(if ((:sendfn db) % storage) 1 0) (:nodes db)))
  )
)
