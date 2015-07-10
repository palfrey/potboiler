(ns potboiler.core
  (:gen-class)
  (:require
   [reloaded.repl :refer [system init start stop go reset]]
   [potboiler.systems :refer [prod-system]]
   [clojure.tools.logging :as log]))

(defn -main
  "Start a production system."
  [& args]
  (reloaded.repl/set-init! prod-system)
  (go))

;; http://stuartsierra.com/2015/05/27/clojure-uncaught-exceptions
(Thread/setDefaultUncaughtExceptionHandler
 (reify Thread$UncaughtExceptionHandler
   (uncaughtException [_ thread ex]
     (log/error ex "Uncaught exception on" (.getName thread)))))
