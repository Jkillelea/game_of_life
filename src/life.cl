__kernel void life(__global char *board, __global char *next_board) {
    int width = get_global_size(0);

    int col = get_global_id(0); // x
    int row = get_global_id(1); // y

    char this_cell = board[row*width + col];

    char sum_neighbors = board[(row+1)*width + (col+0)]
                       + board[(row+1)*width + (col+1)]
                       + board[(row+0)*width + (col+1)]
                       + board[(row-1)*width + (col+1)]
                       + board[(row-1)*width + (col+0)]
                       + board[(row-1)*width + (col-1)]
                       + board[(row+0)*width + (col-1)]
                       + board[(row+1)*width + (col-1)];

    if (this_cell != 0) {
        if (sum_neighbors == 2 || sum_neighbors == 3)
            next_board[row*width + col] = 1;
        else
            next_board[row*width + col] = 0;
    } else {
        if (sum_neighbors == 3)
            next_board[row*width + col] = 1;
        else
            next_board[row*width + col] = 0;
    }

    // next_board[row*width + col] = sum_neighbors;
}