(ns potboiler.tree
  (:require
   [pandect.algo.sha256 :refer :all]
   [clj-leveldb :as level]
   [puget.printer :as puget]
   [clojure.edn :as edn]
   [taoensso.timbre :as timbre]
  )
)

(defmacro def- [id value]
  `(def ^{:public false} ~id ~value)
)

(def- master "MASTER")

(def- start "START")

(def id "ID")

(defn set-node-sender [db newfn]
  (assoc db :send-to-node newfn)
)

(defn set-client-sender [db newfn]
  (assoc db :send-to-client newfn)
)

(defn add-client [db client-id]
  (assoc db :clients (conj (:clients db) client-id))
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
      {:db db :nodes {} :clients []
        :send-to-node (fn [dest msg]
          (timbre/infof "Sending message %s to %s" msg dest)
          true
        )
        :send-to-client (fn [dest msg]
          (timbre/infof "Sending message %s to %s" msg dest)
          true
        )
        :recvfn (fn [src msg]
          (timbre/infof "Receiving message %s from %s" msg src)
        )
      }
    )
  )
)

(defn send-to-node [db dest msgtype msg]
  (timbre/debugf "Sending %s -> %s %s %s" (dbid db) dest msgtype msg)
  ((:send-to-node db) dest (assoc msg :sender (dbid db) :msgtype msgtype))
)

(defn send-to-client [db dest action data]
  (timbre/debugf "Sending %s %s %s" dest action data)
  ((:send-to-client db) dest {:action action :data data})
)

(defn additem [db value]
  (let [canonvalue (tostr value)
        valuehash (hashfn canonvalue)
        oldmaster (masterkey db)
        storage {:hash valuehash :value canonvalue :parent oldmaster}
        masterhash (sha256 (str valuehash oldmaster))
        ]
    (timbre/debugf "%s: Adding '%s' resulting in new masterhash %s" (dbid db) value masterhash)
    (level/put (getdb db)
               masterhash (tostr storage)
               master masterhash)
    (+
      (reduce + 0 (map #(if (send-to-node db % :new-commit (assoc storage :master masterhash)) 1 0) (filter #(not= (dbid db) %) (:nodes db))))
      (reduce + 0 (map #(if (send-to-client db % :apply value) 1 0) (:clients db)))
    )
  )
)

(defn add-from-other [db msg]
  (case (:msgtype msg)
    :new-commit
      (cond
        (= (masterkey db) (:masterhash msg))
          (timbre/debugf "Ignoring applied message %s" msg)
        (-> (getobject db (:masterhash msg)) nil? not)
          (timbre/debugf "%s: Ignoring already applied msg %s" (dbid db) msg)
        (= (masterkey db) (:parent msg))
          (additem db (-> msg :value fromstr))
        :default
          (send-to-node db (:sender msg) :current-master (assoc (getobject db (masterkey db)) :master (masterkey db)))
      )
    :current-master
      (if (-> (getobject db (:master msg)) nil? not)
        (do
          (log/debugf "Have master %s %s" (:master msg) msg)
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
                (doall (map #(send-to-node db (:sender msg) :new-commit %) msgs))
                )))
          )
        )
        (timbre/infof "%s: Need more messages %s" (dbid db) msg)
      )
    (timbre/errorf "%s: Unknown message type: %s %s" (dbid db) (:msgtype msg) msg)
  )
)

(defn history [db]
  (loop [data [] current (masterkey db)]
    (if (= current (startkey db))
      data
      (recur (cons (getitem db current) data) (parentkey db current))
    )
  )
)
