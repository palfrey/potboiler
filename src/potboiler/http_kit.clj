(ns potboiler.http-kit
  (:require [com.stuartsierra.component :as component]
            [org.httpkit.server :refer [run-server]]))

(defrecord WebServer [config port server handler db]
  component/Lifecycle
  (start [component]
    (let [server (run-server (handler {:db db}) {:port port})]
      (assoc component :server server)))
  (stop [component]
    (when server
      (server)
      component)))

(defn new-web-server
  [port handler]
  (map->WebServer {:port port :handler handler}))
