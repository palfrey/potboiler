(ns potboiler.core
  (:gen-class)
  (:require
   [reloaded.repl :refer [system init start stop go reset]]
   [potboiler.systems :refer [prod-system]]
   [taoensso.timbre :as timbre]
   [clojure.string :as str]))

(defn -main
  "Start a production system."
  [& args]
  (reloaded.repl/set-init! prod-system)
  (go))

;; http://stuartsierra.com/2015/05/27/clojure-uncaught-exceptions
(Thread/setDefaultUncaughtExceptionHandler
 (reify Thread$UncaughtExceptionHandler
   (uncaughtException [_ thread ex]
     (timbre/error ex "Uncaught exception on" (.getName thread)))))

(defn default-output-fn
  "Default (fn [data]) -> string output fn.
  You can modify default options with `(partial default-output-fn <opts-map>)`."
  ([data] (default-output-fn nil data))
  ([{:keys [no-stacktrace? stacktrace-fonts] :as opts} data]
   (let [{:keys [level ?err_ vargs_ msg_ ?ns-str hostname_ timestamp_]} data]
     (str
       (force timestamp_) " "
       (str/upper-case (name level))  " "
       "[" (or ?ns-str "?ns") "] - "
       (force msg_)
       (when-not no-stacktrace?
         (when-let [err (force ?err_)]
           (str "\n" (timbre/stacktrace err opts))))))))

(timbre/merge-config! {:level :info :output-fn default-output-fn})
