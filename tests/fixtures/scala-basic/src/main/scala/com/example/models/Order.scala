package com.example.models

import com.example.services.OrderService

case class Order(id: Int, customerId: Int, total: Double) {
  def describeWith(service: OrderService, customer: User): String =
    service.summarize(customer, this)
}
