# Changelog

## 0.1.3 : 2021-06-16

* Fix bug in rawPath handling.\
  Since API Gateway decodes percent encoding of rawPath, path containing %20 did not work correctly.
* Remove unused struct fields

## 0.1.2 : 2021-06-15

* Add Rocket support
* Add examples

## 0.1.1 : 2021-06-14

* Add Warp support

## 0.1.0 : 2021-06-14

* Initial version, run Actix Web on AWS Lambda
