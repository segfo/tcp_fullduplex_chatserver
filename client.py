import socket,sys,threading

def worker1(sock):
    try:
        while True:
            s = input()+"\r\n"
            sock.sendall(s.encode('utf-8'))
    except:
        pass

def worker2(sock):
    try:
        while True:
            # ネットワークのバッファサイズは1024。サーバからの文字列を取得する
            data = sock.recv(1024)
            if len(data)==0:
                print("Broken pipe")
                break
            print(data)
    except:
        pass


with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    try:
        # サーバを指定
        sock.connect(('127.0.0.1', 8080))
        # サーバにメッセージを送る
        # スレッドに workder1 関数を渡す
        t1 = threading.Thread(target=worker1,args=(sock,))
        t2 = threading.Thread(target=worker2,args=(sock,))
        # スレッドスタート
        t1.start()
        t2.start()
        print('started')
        t2.join()
    except:
        pass