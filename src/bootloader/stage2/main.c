#include "stdint.h"
#include "stdio.h"

void _cdecl cstart_(uint16_t bootDrive)
{
	int test = 0;
	puts("        @                             @\r\n");
	puts("@@@@@@  @                             @                           @@@@    @@@@@\r\n");
	puts("@    @  @                             @                         @@   @@  @   @@\r\n");
	puts("@    @  @ @@@    @@@@   @@@@  @ @@@   @ @@@    @@@@   @ @@     @@     @  @\r\n");
	puts("@    @  @@  @@  @@  @@  @     @@  @@  @@  @@  @@  @@  @@       @      @@ @@@\r\n");
	puts("@@@@@@  @    @  @    @  @     @    @  @    @  @   @@  @        @      @@   @@@\r\n");
	puts("@       @    @  @    @   @@   @    @  @    @  @    @  @        @      @@     @@\r\n");
	puts("@       @    @  @    @     @  @    @  @    @  @    @  @        @@     @       @\r\n");
	puts("@       @    @  @@  @@  @  @  @@  @@  @    @  @@  @@  @         @@   @@  @   @@\r\n");
	puts("@       @    @   @@@@   @@@@  @@@@@   @    @   @@@@   @          @@@@     @@@@\r\n");
    
	puts("Test\r\n");
	// printf("Test %d %c %s yadadsasd\r\n",1234, 'c', "balls");
	for(;;) {
		printf("Iteration: %d\r\n", test);
		test++;
	}
}