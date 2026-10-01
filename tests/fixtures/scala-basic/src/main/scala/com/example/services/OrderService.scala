package com.example.services

import com.example.models.{Order, User}

class OrderService {
  def summarize(user: User, order: Order): String =
    s"${user.name} ordered item ${order.id}"
}
