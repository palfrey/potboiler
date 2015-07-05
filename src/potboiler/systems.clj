(ns potboiler.systems
  (:require [system.core :refer [defsystem]]
            (system.components
             [http-kit :refer [new-web-server]]
             [repl-server :refer [new-repl-server]])
            [environ.core :refer [env]]
            [potboiler.leveldb :refer [new-database]]
            [potboiler.handler :refer [app]]))

(defsystem dev-system
  [:web (new-web-server (Integer. (env :http-port)) app)
   :db (new-database (env :db-path))])

(defsystem prod-system
  [:web (new-web-server (Integer. (env :http-port)) app)
   :db (new-database (env :db-path))
   :repl-server (new-repl-server (Integer. (env :repl-port)))])
