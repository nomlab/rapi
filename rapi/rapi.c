#include "rapi.h"

#include <mpi.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
struct timespec ct1, ct6;
struct timespec rt1, rt6;

double nsec_to_sec(time_t nsec) { return (double)nsec / (1000 * 1000 * 1000); }

double timespec_to_sec(time_t sec, time_t nsec) {
    return (double)sec + nsec_to_sec(nsec);
}

double calc_elapsed_time(struct timespec start, struct timespec end) {
    return timespec_to_sec(end.tv_sec - start.tv_sec, end.tv_nsec - start.tv_nsec);
}

// Count up the number of reveived SIGCONT
volatile sig_atomic_t num_sigcont = 0;
void sigcont_handler(int signum) { num_sigcont += 1; }

int MPI_Init(int *argc, char ***argv) {
    int ret;
    pid_t pid;
    int fd;

    pid = getpid();
    fd = create_udp_socket();
    if (fd == -1) {
        fprintf(stderr, "RAPI ERROR: creating socket failed\n");
        exit(1);
    }
    ret = send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                            (struct Request){.t = REQ_REGISTER, .pid = pid});
    if (ret == -1) {
        fprintf(stderr, "RAPI ERROR: sending request failed\n");
        exit(1);
    }

    // Insert handler for SIGCONT
    signal(SIGCONT, sigcont_handler);

    clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &ct1);
    clock_gettime(CLOCK_REALTIME, &rt1);

    ret = PMPI_Init(argc, argv);

    return ret;
}

int MPI_Finalize() {
    int ret;
    pid_t pid;
    int fd;

    int rank;
    MPI_Comm_rank(MPI_COMM_WORLD, &rank);

    ret = PMPI_Finalize();
    clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &ct6);
    clock_gettime(CLOCK_REALTIME, &rt6);
    printf("%d, %f, %f, %d\n", rank, calc_elapsed_time(rt1, rt6),
           calc_elapsed_time(ct1, ct6), num_sigcont);

    pid = getpid();
    fd = create_udp_socket();
    if (fd == -1) {
        fprintf(stderr, "RAPI ERROR: creating or binding socket failed\n");
        exit(1);
    }
    ret = send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                            (struct Request){.t = REQ_UNREGISTER, .pid = pid});
    if (ret == -1) {
        fprintf(stderr, "RAPI ERROR: sending request failed\n");
        exit(1);
    }

    return ret;
}

int MPI_Send(const void *buf, int count, MPI_Datatype datatype, int dest, int tag,
             MPI_Comm comm) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Send(buf, count, datatype, dest, tag, comm);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int MPI_Recv(void *buf, int count, MPI_Datatype datatype, int source, int tag,
             MPI_Comm comm, MPI_Status *status) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Recv(buf, count, datatype, source, tag, comm, status);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int MPI_Sendrecv(const void *sendbuf, int sendcount, MPI_Datatype sendtype, int dest,
                 int sendtag, void *recvbuf, int recvcount, MPI_Datatype recvtype,
                 int source, int recvtag, MPI_Comm comm, MPI_Status *status) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Sendrecv(sendbuf, sendcount, sendtype, dest, sendtag, recvbuf, recvcount,
                        recvtype, source, recvtag, comm, status);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int MPI_Alltoall(const void *sendbuf, int sendcount, MPI_Datatype sendtype, void *recvbuf,
                 int recvcount, MPI_Datatype recvtype, MPI_Comm comm) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Alltoall(sendbuf, sendcount, sendtype, recvbuf, recvcount, recvtype, comm);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int MPI_Wait(MPI_Request *request, MPI_Status *status) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Wait(request, status);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int MPI_Waitall(int count, MPI_Request array_of_requests[],
                MPI_Status *array_of_statuses) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Waitall(count, array_of_requests, array_of_statuses);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int MPI_Allreduce(const void *sendbuf, void *recvbuf, int count, MPI_Datatype datatype,
                  MPI_Op op, MPI_Comm comm) {
    int ret;
    int fd;

    fd = create_udp_socket();
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_BEGIN_COMM, .pid = 0});
    ret = PMPI_Allreduce(sendbuf, recvbuf, count, datatype, op, comm);
    send_req_to_rapid(fd, htonl(INADDR_LOOPBACK), get_rapid_port(),
                      (struct Request){.t = REQ_END_COMM, .pid = 0});
    close(fd);

    return ret;
}

int create_udp_socket() {
    int fd;

    fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd == -1)
        return -1;

    return fd;
}

in_port_t get_rapid_port() {
    uint16_t port_host_order;
    char *rapid_port_env = getenv("RAPID_PORT");
    if (rapid_port_env == NULL) {
        port_host_order = RAPID_DEFAULT_PORT;
    } else {
        port_host_order = atoi(rapid_port_env);
    }
    return htons(port_host_order);
}

int send_req_to_rapid(int fd, in_addr_t rapid_addr, in_port_t rapid_port,
                      struct Request req) {
    ssize_t n_sent;
    struct sockaddr_in saddr;

    saddr.sin_family = AF_INET;
    saddr.sin_addr.s_addr = rapid_addr;
    saddr.sin_port = rapid_port;
    n_sent = sendto(fd, (char *)&req, RAPID_REQUEST_SIZE, 0, (struct sockaddr *)&saddr,
                    sizeof(saddr));
    if (n_sent == -1)
        return -1;

    return n_sent;
}
