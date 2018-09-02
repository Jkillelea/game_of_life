__kernel void life(__global unsigned char *const board, 
                   __global unsigned char *next_board) {
    unsigned int width = get_global_size(0); // board width
    unsigned int col   = get_global_id(0);   // x
    unsigned int row   = get_global_id(1);   // y

    unsigned char this_cell = board[row*width + col];

    // should gracefully handle out of bounds access
    unsigned char sum_neighbors = board[(row+1)*width + (col+0)]
                                + board[(row+1)*width + (col+1)]
                                + board[(row+0)*width + (col+1)]
                                + board[(row-1)*width + (col+1)]
                                + board[(row-1)*width + (col+0)]
                                + board[(row-1)*width + (col-1)]
                                + board[(row+0)*width + (col-1)]
                                + board[(row+1)*width + (col-1)];

    // cell is alive
    if (this_cell != 0) {
        if (sum_neighbors == 2 || sum_neighbors == 3)
            next_board[row*width + col] = 1;
        else
            next_board[row*width + col] = 0;
    } else { // cell is dead
        if (sum_neighbors == 3)
            next_board[row*width + col] = 1;
        else
            next_board[row*width + col] = 0;
    }
}

