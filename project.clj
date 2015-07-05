(defproject potboiler "0.1.0-SNAPSHOT"
  :description "FIXME: write description"
  :url "http://example.com/FIXME"
  :license {:name "Eclipse Public License"
            :url "http://www.eclipse.org/legal/epl-v10.html"}
  :dependencies [[org.clojure/clojure "1.6.0"]
                 [ring "1.3.1"]
                 [ring/ring-defaults "0.1.2"]
                 [compojure "1.2.0"]
                 [org.danielsz/system "0.1.1"]
                 [environ "1.0.0"]
                 [factual/clj-leveldb "0.1.1"]
                 [http-kit "2.1.19"]
                 [de.ubercode.clostache/clostache "1.4.0"]]
  :plugins [[lein-environ "1.0.0"]]
  :offline? true
  :profiles {:dev {:source-paths ["dev"]
                   :env {:http-port 3000
                         :db-path "/tmp/level-db"}}
             :prod {:env {:http-port 8000
                          :repl-port 8001
                          :db-path "/tmp/level-db"}
                    :dependencies [[org.clojure/tools.nrepl "0.2.5"]]}
             :uberjar {:aot :all}}
  :main ^:skip-aot potboiler.core
  :target-path "target/%s")
