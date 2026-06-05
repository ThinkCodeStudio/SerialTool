
## Function
* RX TX(show TX)
    * show HEX
    * Command List
* Stream
* Ymodem Send & Receive
* Chart
* modbus rtu

修改方案, 分三个线程, 使用锁共享状态结构体, 一个线程只刷新UI, 一个线程处理键盘输入, 串口后台接收线程, 设计任务状态机.
设计思想参考MVC