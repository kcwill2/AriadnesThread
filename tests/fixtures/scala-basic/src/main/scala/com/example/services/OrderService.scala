package com.example.services

import com.example.models.{Order, User}
import scala.collection.mutable._

class OrderService {
  val buffer = ArrayBuffer[Int]()

  def record(orderId: Int): Unit = {
    buffer += orderId
  }

  def summarize(user: User, order: Order): String =
    s"${user.name} ordered item ${order.id}"
}
