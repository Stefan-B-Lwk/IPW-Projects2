#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <stdint.h>
#include <unistd.h>

#define STATE_OK 1
#define STATE_FAIL 166
#define STATE_GENERAL_ERROR 255
#define STATE_STILL_NOT_OK 95

void func(uint8_t state, int r, int state_check){
	//if ( r % 2 != 0 ){
		if ( state == STATE_OK && state == state_check){
		printf("Executing very sensitive code...\n");
		}
	//}
}

int main(){
	srand(time(NULL));
	int r = rand();

	uint8_t state = STATE_GENERAL_ERROR;
	uint8_t state_check = STATE_GENERAL_ERROR;
	if(r % 4 == 2){
		state ^= 2;
	}

	float wait;
	r = r % 2000;
	wait = (float)r / 1000;
	printf("%f\n",wait);
	sleep(wait);

	r = rand();
	if(r % 4 == 2){
		state_check ^= 2;
	}
	
	printf("%d%d",state,state_check);

	if(STATE_OK == state){
		
		func(state,r, state_check);
	}else{
		if(r % 4 == 3){
			
			func(state,r, state_check);
		}
	}
	return 0;
}