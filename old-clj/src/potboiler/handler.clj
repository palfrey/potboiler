(ns potboiler.handler
  (:require
   [compojure.route :as route]
   [compojure.core :refer [defroutes GET POST ANY context routes]]
   [ring.middleware.defaults :refer [wrap-defaults site-defaults api-defaults]]
   [potboiler.html :as html]
   [reloaded.repl :refer [system]]
   [cheshire.core :as json]
   [pandect.algo.sha256 :refer :all]
   [clj-leveldb :as level]
   [puget.printer :as puget]))

(defroutes frontend
  (GET "/" [] (html/index))
)

(defn json-response [data & [status]]
  {:status  (or status 200)
   :headers {"Content-Type" "application/json; charset=utf-8"}
   :body    (json/generate-string data)})


; json/parse-string

; json/generate-string
; sha256

(defn conn [] (:connection (:db system)))

(defn nil-or-value [value default]
  (if (nil? value)
    default
    value
  )
)

(defn default-array [value]
  (nil-or-value value [])
)

(defn uuid [] (str (java.util.UUID/randomUUID)))

(defroutes api
  (context "/api" []
    (GET "/keys" []
      (-> (conn) level/iterator default-array json-response)
    )
    (GET "/key/:key" [key]
      (json-response (level/get (conn) key))
    )
    (POST "/key" [:as {body :body}]
      (let [data (-> body slurp puget/pprint-str)
            key (sha256 data)]
        (level/put (conn) key data)
        (json-response key)
      )
    )
    (ANY "*" []
      (route/not-found "Not found API")
    )
  )
)

(defroutes not-found
  (route/not-found "Not found")
)

(defn wrap-services [f services]
  (fn [req]
    (f (assoc req :services services))))

(defn app [services]
   (-> (routes frontend api not-found)
      (wrap-services services)
  ))
