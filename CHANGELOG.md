# Changelog

## 0.1.6 : Not yet released

* Replace lamedh\_runtime with lambda\_runtime 0.4.0
* Update Actix-Web to 4.0.0-beta.8

## 0.1.5 : 2021-06-21

* Update Actix-Web to 4.0.0-beta.7

## 0.1.4 : 2021-06-21

* Support transparent Brotli compression

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
