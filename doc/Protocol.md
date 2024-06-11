# rapi 通信プロトコル
通信は経路は以下の4種である．
1. rapi -> rapid
2. rapid -> rapictld
3. rapictld -> rapid
4. rapid -> rapi

1~3 については UDP で以下の内容を送信する．
```rust
struct Request {
    req: ReqType,
    pid: c_int,
}

enum ReqType {
    Unregister = 0,
    Register = 1,
    Stop = 2,
    Cont = 3,
    CommBegin = 4,
    CommEnd = 5,
}
```
以降では，各通信の内容と契機について説明する．

## rapi -> rapid
### 通信契機
MPI 関数が開始した際に，`Request`を送信する．
対象とする MPI 関数は以下の通りである．
* MPI_Init
* MPI_Init_thread
* MPI_Finarize
* MPI_Send
* MPI_Recv
* MPI_Sendrecv
* MPI_Alltoall
* MPI_Wait
* MPI_Waitall
* MPI_Allreduce

### 通信内容
通信内容は以下の3種類である．
1. MPI プログラムの開始を通知  
    rapi は，`ReqType = Register`，`pid` を自身の PID として，`Request` を rapid に送信する．
    対象とする MPI 関数は MPI_Init と MPI_Init_thread である．
    rapid は受け取った `Request` の PID を管理対象として登録する．
2. MPI プログラムの終了を通知  
    rapi は，`ReqType = Unregister`，`pid` を自身の PID として，`Request` を rapid に送信する．
    対象とする MPI 関数は MPI_Finarize である．
    rapid は受け取った `Request` の PID を管理対象から削除する．
3. MPI 通信関数の開始/終了を通知  
    rapi は，`ReqType = CommBegin`，または`ReqType = CommBegin`，`pid` は 0 (ダミーデータ) として`Request` を rapid に送信する．
    対象とする MPI 関数は 1,2 に示したもの以外の対象MPI関数である．

## rapid -> rapictld
### 通信契機
rapid が rapi から `Request` を受信した際．

### 通信内容
受信したものと同一の `Request` を rapictld に送信する．

## rapictld -> rapid
### 通信契機
収集した MPI プログラムの状態を考慮し，MPI プログラムを停止/再開すべきと判断した際．

### 通信内容
rapictld は `ReqType = Stop`，または `ReqType = Cont`，`pid` を 0 (ダミーデータ) として `Request` を rapid に送信する．

## rapid -> rapi
### 通信契機
rapid が rapictld から `Stop` または `Cont` を受信した際．

### 通信内容
rapid は Linux カーネルの `signal` システムコールを用いて，管理対象の全プロセス (MPIプログラム) にシグナルを送信する．
送信するシグナルは以下の2種類である．
1. SIGSTOP  
    `Stop` を受け取った際に，プロセスを停止するために送信する．
2. SIGCONT  
    `Cont` を受け取った際に，プロセスを再開するために送信する．
