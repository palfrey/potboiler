(ns potboiler.core-test
  (:use midje.sweet)
  (:require [clojure.test :refer :all]
            [potboiler.core :refer :all]
            [puget.printer :as puget]
            [pandect.algo.sha256 :refer :all]))

(fact "puget does key ordering ok"
  (puget/pprint-str {:a "foo" :b "bar"}) => (puget/pprint-str {:b "bar" :a "foo"})
)

(fact "hash works"
  (-> {:a "foo" :b "bar" :c ["wibble"]} puget/pprint-str sha256) => "50e9847c0aa2a3555af33e9e497029674bdfe7f002293da8d3402ee68af3da1b"
)
