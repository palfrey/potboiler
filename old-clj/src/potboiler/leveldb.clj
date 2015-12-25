(ns potboiler.leveldb
  (:require [com.stuartsierra.component :as component]
            [clj-leveldb :as level]))

(defrecord LevelDB [db-path connection]
  component/Lifecycle

  (start [component]
    (let [conn (level/create-db
                (:db-path component)
                {
                 :key-decoder byte-streams/to-string
                 :val-decoder byte-streams/to-string
                 })]
      (assoc component :connection conn)))

  (stop [component]
    (.close connection)
    (assoc component :connection nil)))

(defn new-database
  ([db-path]
    (LevelDB. db-path nil)))
