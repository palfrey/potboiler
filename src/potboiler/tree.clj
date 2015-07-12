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

(defn set-receiver [db newfn]
  (assoc db :recvfn newfn)
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
  (-> (level/get (getdb db) key) fromstr)
)

(defn getitem [db key]
  (-> (getobject db key) :value fromstr)
)

(defn parentkey [db key]
  (-> (getobject db key) :parent)
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
       :recvfn (fn [src msg]
          (log/infof "Receiving message %s from %s" msg src)
        )
      }
    )
  )
)

(defn sendmsg [db dest msgtype msg]
  ((:sendfn db) dest (assoc msg :sender (dbid db) :msgtype msgtype))
)

(defn additem [db value]
  (let [canonvalue (tostr value)
        valuehash (hashfn canonvalue)
        oldmaster (masterkey db)
        storage {:hash valuehash :value canonvalue :parent oldmaster}
        masterhash (sha256 (str valuehash oldmaster))
        ]
    (log/debugf "Adding '%s' resulting in new masterhash %s" value masterhash)
    (level/put (getdb db)
               masterhash (tostr storage)
               master masterhash)
    (reduce + 0 (map #(if (sendmsg db % :new-commit (assoc storage :master masterhash)) 1 0) (filter #(not= (dbid db) %) (:nodes db))))
  )
)

(defn add-from-other [db msg]
  (case (:msgtype msg)
    :new-commit
      (cond
        (= (masterkey db) (:master msg))
          (log/debugf "Ignoring applied message %s" msg)
        (= (masterkey db) (:parent msg))
          (additem db (-> msg :value fromstr))
        :default
          (sendmsg db (:sender msg) :current-master (assoc (getobject db (masterkey db)) :master (masterkey db)))
      )
    :current-master
      (if (-> (getobject db (:master msg)) nil? not)
        (do
          (log/infof "Have master %s %s" (:master msg) msg)
          (let [masterhash (masterkey db)]
            (-> (loop [msgs []
                   current masterhash]
              (cond
                (= (:master msg) current)
                  msgs
                (= (startkey db) current)
                  (do
                    (log/errorf "Hit startkey. Possible different start key?")
                    []
                  )
                :default
                  (recur (cons (assoc (getobject db current) :master masterhash) msgs) (parentkey db current))
              )
            ) ((fn [msgs]
                (doall (map #(sendmsg db (:sender msg) :new-commit %) msgs))
                )))
          )
        )
        (log/errorf "Can't find master %s" (:master msg))
      )
    (log/errorf "Unknown message type: %s %s" (:msgtype msg) msg)
  )
)
