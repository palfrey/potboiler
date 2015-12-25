(ns potboiler.tree
  (:require
   [pandect.algo.sha256 :refer :all]
   [clj-leveldb :as level]
   [puget.printer :as puget]
   [clojure.edn :as edn]
   [taoensso.timbre :as timbre]
  )
)

(defmacro non-lazy-for [& body]
  `(doall (for ~@body))
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

(defn add-neighbour [db neighbour-id master-hash]
  (assoc db :nodes (assoc (:nodes db) neighbour-id {:id neighbour-id :master master-hash}))
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
        masterhash (sha256 (str valuehash oldmaster))
        storage {:masterhash masterhash :valuehash valuehash :value canonvalue :parent oldmaster}
        ]
    (timbre/debugf "%s: Adding '%s' resulting in new masterhash %s" (dbid db) value masterhash)
    (level/put (getdb db)
               masterhash (tostr storage)
               master masterhash)
    (+
      (reduce + 0 (map #(if (send-to-node db % :new-commit (assoc storage :master masterhash)) 1 0) (filter #(not= (dbid db) %) (keys (:nodes db)))))
      (reduce + 0 (map #(if (send-to-client db % :apply value) 1 0) (:clients db)))
    )
  )
)

(defn update-to-current [db dest dest-master]
  (let [masterhash (masterkey db)]
    (-> (loop
          [msgs []
           current masterhash]
          (cond
            (= dest-master current)
              msgs
            (= (startkey db) current)
              (do
                (timbre/errorf "Hit startkey. Possible different start key for %s?" dest)
                []
              )
            :default
              (recur (cons (assoc (getobject db current) :master masterhash) msgs) (parentkey db current))
          )
        ) ((fn [msgs]
             (doall (map #(send-to-node db dest :new-commit %) msgs))
             )))
  )
)

(defn greatest [x y]
  (if (= -1 (compare x y)) ; y only goes first if x is less. equal returns x y order
    y
    x
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
          (timbre/debugf "Have master %s %s" (:master msg) msg)
          (update-to-current db (:sender msg) (:master msg))
        )
        (send-to-node db (:sender msg) :ancestor-commit (assoc (getobject db (parentkey db (masterkey db))) :master (masterkey db)))
        ;(timbre/errorf "%s: Can't find master %s" (dbid db) (:master msg))
      )
    :ancestor-commit
      (if (-> (getobject db (:masterhash msg)) nil? not)
        (do
          (timbre/infof "%s: Have common ancestor %s" (dbid db) (:masterhash msg))
          (let [winning-id (greatest (dbid db) (:sender msg))
                local-win (= winning-id (dbid db))
                ]
            (timbre/infof "%s: Won? %s" (dbid db) local-win)
            (send-to-node db (:sender msg) :common-ancestor {:winner winning-id :common-ancestor (:masterhash msg)})
          )
        )
        (timbre/infof "%s: Need more messages %s" (dbid db) msg)
      )
    (timbre/errorf "%s: Unknown message type: %s %s" (dbid db) (:msgtype msg) msg)
  )
  ;(timbre/debugf "Message %s" msg)
  (assoc-in db [:nodes (:sender msg) :master] (:master msg))
)

(defn history [db]
  (loop [data [] current (masterkey db)]
    (if (= current (startkey db))
      data
      (recur (cons (getitem db current) data) (parentkey db current))
    )
  )
)

(defn resync [db]
  (non-lazy-for [node (vals (:nodes db))]
    (if (not= (:master node) (masterkey db))
      (do
        (timbre/infof "%s: %s is out of date, so updating" (dbid db) (:id node))
        (update-to-current db (:id node) (:master node))
      )
    )
  )
)
