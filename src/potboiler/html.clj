(ns potboiler.html
  (:require 
   [clostache.parser :as clo]
   ))

(defn index []
  (clo/render-resource "templates/index.mustache" {}))
