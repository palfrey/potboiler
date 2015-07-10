(ns potboiler.systems
  (:require [com.stuartsierra.component :as component]
            (system.components
             [repl-server :refer [new-repl-server]])
            [environ.core :refer [env]]
            [potboiler.leveldb :refer [new-database]]
            [potboiler.handler :refer [app]]
            [potboiler.http-kit :refer [new-web-server]]))

(defn dev-system []
  (component/system-map
     :db (new-database (env :db-path))

   :web (component/using
          (new-web-server (Integer. (env :http-port)) app)
          [:db])
  )
)

(defn prod-system []
  (component/system-map
  :db (new-database (env :db-path))
   :web (component/using
         (new-web-server (Integer. (env :http-port)) app)
         [:db]
         )
   :repl-server (new-repl-server (Integer. (env :repl-port)))))
