#include <stdio.h>
#include <time.h>
#include <string.h>
#include "timing.h"
#include <unistd.h>


int check_with_good_compare(char string[15]){
	
	int value = 0;

	clock_t start = clock();
	const char secret[] = "PlayboiCarti";
    
    // Perform constant-time comparison
    for (int i = 0; i < 15; i++) {
        if( string[i] != secret[i] )
			value = 1;
    }

	clock_t end = clock();

	if ((float)(end - start) / CLOCKS_PER_SEC < 0.0001 )
		usleep((__useconds_t)(0.0001 - (float)(end - start) / CLOCKS_PER_SEC)*1000000000 );
	if (value == 1 ){
		return 0;
	}

	return 1;
}



int main(){
	float max[15];
	for ( int j = 0 ; j < 15; j++)
		max[j] = 0;
	char max_val[15];
	int i = 65;
	char caract_nou;
	char string[15];
	char string_final[15]= {0};
	//string_final[0]='A';
	for ( int j = 0; j < 15 ; j++){
		i = 65;
		while ( 1 ){
			strcpy(string,string_final);
			string[j] = i;
			clock_t start = clock();
			int one_for_success = check_with_good_compare(string);
			clock_t end = clock();
			printf("Timp: %f\n", (float)(end - start) / CLOCKS_PER_SEC);
			//printf("Result %d in %f seconds.\n", one_for_success, (float)(end - start) / CLOCKS_PER_SEC);
			if ( ((float)(end - start) / CLOCKS_PER_SEC) > max[j] ){
				max[j] = (float)(end - start) / CLOCKS_PER_SEC;
				string_final[j] = i;
			}

			i++;
			if ( i == 91){
				i = 97;
			}
			if ( i == 123)
				break;

		}
		printf("%c",string_final[j]);
	}
	string_final[14]='\0';
	printf("%s",string_final);
	// clock_t start = clock();
	// int one_for_success = check_with_bad_compare("DoNotDecompile");
	// clock_t end = clock();
	// printf("Result %d in %f seconds.\n", one_for_success, (float)(end - start) / CLOCKS_PER_SEC);


	return 0;
}